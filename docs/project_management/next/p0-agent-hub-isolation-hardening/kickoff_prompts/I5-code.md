# Task I5-code (Docs + verification) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I5-spec.md`, and this prompt.
3. Set `I5-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I5-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i5-docs-verify-code
   git worktree add wt/ahih-i5-docs-verify-code ahih-i5-docs-verify-code
   cd wt/ahih-i5-docs-verify-code
   ```

## Spec (shared with I5-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I5-spec.md`

## Scope & Guardrails
- Production code + docs tied to verification tooling (no tests).
- Align docs to real guarantees and add a minimal verification checklist/script.
- If scripts are changed/added, keep them shellcheck-clean.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
git ls-files '*.sh' | xargs -r shellcheck -x -S warning
```

## End Checklist
1. Confirm required checks are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I5-code`).
5. Remove worktree.

