# Task I4-code (Landlock optional layer) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I4-spec.md`, and this prompt.
3. Set `I4-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I4-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i4-landlock-code
   git worktree add wt/ahih-i4-landlock-code ahih-i4-landlock-code
   cd wt/ahih-i4-landlock-code
   ```

## Spec (shared with I4-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I4-spec.md`

## Scope & Guardrails
- Production code only (no tests).
- Add Landlock detection + enforcement as optional layer/fallback per spec; surface in doctor output.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```

## End Checklist
1. Confirm fmt/clippy are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I4-code`).
5. Remove worktree.

